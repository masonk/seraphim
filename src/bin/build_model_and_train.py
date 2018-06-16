from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

from pathlib import Path
import os
import tensorflow as tf
import glob

num_epochs = 100
minibatch_size = 128
dataset_dir = "src/tictactoe/gamedata"

model_id = "01"
model_dir = "src/tictactoe/saved_models"
saver_prefix = model_dir + "/model_{}".format(model_id)
graph_filename = "{}_graph.pb".format(saver_prefix)

def make_dataset(minibatch_size, dataset_dir):
    files = glob.glob("{}/*.tfrecord".format(dataset_dir))
    print("loading", files)
    dataset = tf.data.TFRecordDataset(files)
    dataset = dataset.map(parse)
    dataset = dataset.shuffle(buffer_size=100000)
    dataset = dataset.batch(minibatch_size)
    print("loaded data")
    return dataset

def parse(bytes):
  features = {"game": tf.FixedLenFeature((), tf.string),
              "choice": tf.FixedLenSequenceFeature((), tf.float32, allow_missing=True)}
  parsed_features = tf.parse_single_example(bytes, features)
  game = tf.decode_raw(parsed_features["game"], tf.uint8)
  choice =  parsed_features["choice"]
  return tf.reshape(game, [19]), tf.reshape(choice, [9])

dataset = make_dataset(minibatch_size, dataset_dir)
iterator = dataset.make_initializable_iterator()
example, label = iterator.get_next()

dense = tf.layers.dense(tf.cast(example, tf.float32), units=64, activation=tf.nn.relu)
logits = tf.layers.dense(dense, units=9, activation=tf.nn.relu)
softmax = tf.nn.softmax(logits, name='softmax')

sess = tf.Session()
init = tf.group(
        tf.global_variables_initializer(), 
        tf.local_variables_initializer(), 
        iterator.initializer)

loss = tf.losses.mean_squared_error(labels=label, predictions=softmax)
optimizer = tf.train.GradientDescentOptimizer(.01)
train = optimizer.minimize(loss, name='train')

saver = tf.train.Saver(None, name="saver")

definition = tf.Session().graph_def
tf.train.write_graph(definition, model_dir, "graph_{}.pb".format(model_id), as_text=False)

sess.run(init)

for i in range(num_epochs):
    sess.run(iterator.initializer)
    
    while True:
        try:
            sess.run(train)
        except tf.errors.OutOfRangeError:
            break
    save_path = saver.save(sess, saver_prefix)
    print("Model saved in path: %s" % save_path)
