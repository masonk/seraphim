use crate::error::{ModelError, TensorflowError};
use failure::ResultExt;
use tensorflow as tf;

#[derive(Debug)]
pub struct Priors {
    pub ps: Vec<f32>,
    pub q: f32,
}

// The first float is expected to be the qvalue for the position
// The remaining floats are the ps for each move.error
// This method must return exactly game.max_actions() + 1 f32s.
pub trait Inference: std::fmt::Debug {
    fn infer(&mut self, input: &[u8]) -> crate::error::Result<Priors>;
}

// A type which does inference via Tensorflow Rust bindings
#[derive(Debug)]
pub struct TensorFlowInferenceEngine {
    session: tf::Session,
    example: tf::Operation,
    output: tf::Operation,
    graph: tf::Graph,
    training: Option<tf::Operation>,
    timeout: std::time::Duration,
    max_batch_size: usize,
    training_tensor: tf::Tensor<bool>,
}

impl TensorFlowInferenceEngine {
    pub fn from_saved_model<P>(
        p: P,
        max_batch_size: usize,
        timeout: std::time::Duration,
    ) -> crate::error::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        trace!("Attemping to load saved model from '{:?}'", p.as_ref());
        let mut graph = tf::Graph::new();

        let _tags: [&str; 0] = [];
        let session = tf::Session::from_saved_model(
            &tf::SessionOptions::new(),
            &["serve"],
            &mut graph,
            p.as_ref(),
        )
        .map_err(|tf| ModelError::CouldntLoad {
            dir: p.as_ref().to_owned(),
            tf: tf.into(),
        })?;
        trace!("...success.");

        let example = graph
            .operation_by_name_required("example")
            .map_err(TensorflowError::from)
            .context("Expected a graph op named 'example'.")?;
        let softmax = graph
            .operation_by_name_required("softmax")
            .map_err(TensorflowError::from)
            .context("Expected a graph op named 'softmax'.")?;
        let training = graph
            .operation_by_name("training")
            .context("Error while extracting the 'training' op.")?;
        let mut training_tensor: tf::Tensor<bool> = tf::Tensor::new(&[]);
        training_tensor[0] = false;
        Ok(TensorFlowInferenceEngine {
            session,
            graph,
            example,
            output: softmax,
            training,
            max_batch_size,
            timeout,
            training_tensor,
        })
    }
}

impl Inference for TensorFlowInferenceEngine {
    fn infer(&mut self, input: &[u8]) -> crate::error::Result<Priors> {
        let tensor = tf::Tensor::new(&[1, input.len() as u64])
            .with_values(&input)
            .map_err(TensorflowError::from)?;
        let mut inference_step = tf::SessionRunArgs::new();

        inference_step.add_feed(&mut self.example, 0, &tensor);
        if let Some(ref mut training) = self.training {
            inference_step.add_feed(training, 0, &self.training_tensor);
        }

        let softmax_output_token = inference_step.request_fetch(&self.output, 0);

        self.session
            .run(&mut inference_step)
            .expect("failed to run inference step");

        let output: tf::Tensor<f32> = inference_step
            .fetch(softmax_output_token)
            .map_err(TensorflowError::from)?;

        let ps = output[1..].iter().map(|v| *v).collect::<Vec<f32>>();
        Ok(Priors { ps, q: output[0] })
    }
}
