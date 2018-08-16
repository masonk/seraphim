#!/usr/bin/env python3

from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

from pathlib import Path
import argparse
import datetime
import glob
import os
import shutil
import signal
import tensorflow as tf

from tensorflow.python.saved_model.signature_constants import REGRESS_METHOD_NAME, PREDICT_METHOD_NAME
from tensorflow.python.saved_model.signature_def_utils import build_signature_def
from tensorflow.python.saved_model.tag_constants import TRAINING, SERVING
from tensorflow.python.saved_model.utils import build_tensor_info

'''
This script initialize a new model, or restores a serialized metagraph, and trains it.

It operates over this file structure:

src/bin/tictactoe
    gamedata/                         [6]
        0.tfrecord                    [7]
        1.tfrecord                
        control                       [8]
        2                             [9]
    models/
        foo/                          [1]   
            champion/                 [2]
                checkpoints/          [3] 
                saved_model/          [4]
            2018-08-13T23:11:51-0/    [5]
                checkpoints/        
                saved_model/          

[1]: A model dir. Every model has a name, and corresponds to a particular configuration of neural network.  
    Each directory in a model is a snapshot of the model at particular point in time.
[2]: 'champion' is a special snapshot, that corresponds to the current best-known player. A separate process continually
     compares snapshots to each other in an eternal tournament. Periodically, the frontrunner of the tournament is upgraded
     to the champion, and training resumes from the champion.
[3]: The metagraph of a snapshot. This is a serialization of the model suitable for resuming training. Files
    in this directory are written and consumed by the TensorflowFramework.
[4]: The SavedModel of a snapshot. This is a serialization of the model suitable for inference ("serving"), 
    but it can't easily be resumed for training. Files in this directory are written and consumed by the TensorFlow framework.
[5]: Snapshots, other than the special "champion", are named according to the date and time when they were created, followed
    by the global_step of the model (i.e., the number of minibatches of training that the model has experienced).
[6]: gamedata is generated by games of self-play between two instances the current champion. Each record is a 
    (state, action) pair.
[7]: game records are saved in the TFRecord format by seraphim.
[8]: control file contains metadata about game records. It is used for the correct rotation of game files. At any point in 
    time, only the 50 most .tfrecord files will be present in gamedata.
[9]: while a new tfrecord is in progress, it will not be suffixed with .tfrecord, and should not be trained from. It is not
    guaranteed to be a valid tfrecord.
'''
parser = argparse.ArgumentParser(description='Initialize a TicTacToe expert model.')
parser.add_argument('name', metavar='foo-model', help='Model prefix')
parser.add_argument('--init', dest='init', action='store_true')
parser.set_defaults(init=False)

args = parser.parse_args()

model_dir = "src/tictactoe/models/" + args.name + "/"

# save a new SavedModel to compete in the eternal tournament after running through this many epochs of training
snapshot_epochs = 1
minibatch_size = 128

# take training examples (stored in TFRecord format) from files in this directory:
dataset_dir = "src/tictactoe/gamedata"

def make_dataset(minibatch, dataset_dir):
    files = glob.glob("{}/*.tfrecord".format(dataset_dir))
    # print(files)
    dataset = tf.data.TFRecordDataset(files)
    dataset = dataset.map(parse)

    dataset = dataset.shuffle(buffer_size=10000)
    # dataset = dataset.repeat(1)
    dataset = dataset.apply(tf.contrib.data.batch_and_drop_remainder(minibatch_size))
    # dataset = dataset.batch(minibatch, drop_remainder=True) # v1.10?
    return dataset

def parse(bytes):
  features = {"game": tf.FixedLenFeature((), tf.string),
              "choice": tf.FixedLenSequenceFeature((), tf.float32, allow_missing=True)}
  parsed_features = tf.parse_single_example(bytes, features)
  game = tf.decode_raw(parsed_features["game"], tf.uint8)
  choice =  parsed_features["choice"]
  return tf.reshape(game, [19]), tf.reshape(choice, [9])

def take_snapshot(sess, saver, global_step, snapshot_id=None):
    global_step_val = sess.run(global_step)
    if snapshot_id is None:
        snapshot_id = "{}-{}".format(datetime.datetime.now().replace(microsecond=0).isoformat(), global_step_val)
    snapshot_dir = model_dir + snapshot_id + "/"
    saver_dir = model_dir + snapshot_id + "/checkpoints/"
    saver_prefix = saver_dir + "model"
    saved_model_dir = snapshot_dir + "saved_model"
    try:
        os.makedirs(saver_dir)
    except:
        None

    saver.save(sess, saver_prefix, global_step)
    save_savedmodel(sess, saved_model_dir)

def save_savedmodel(sess, dir):
    # SavedModels are "hermetic" complete representations of the training model, meant for consumption
    # across binary boundaries. They have the unfortunate side effect of not being easy to continue training with.
    # In essence, they're frozen snapshots of the model at a moment in time. We periodically save snapshots
    # every snapshot_epochs number of epochs, and enter the resultant models into the eternal tournament.
    # The best such savedmodels have their corresponding metagraph upgraded into "the current metagraph"
    # for continued training.
    example = tf.get_collection("example")[0]
    label = tf.get_collection("label")[0]
    softmax = tf.get_collection('softmax')[0]
    shutil.rmtree(dir, ignore_errors=True)
    builder = tf.saved_model.builder.SavedModelBuilder(dir)
    training_inputs = {
        "example": build_tensor_info(example),
        "label": build_tensor_info(label)
    }
    serving_inputs = {
        "example": build_tensor_info(example)
    }
    signature_outputs = {
        "softmax": build_tensor_info(softmax)
    }

    training_signature_def = build_signature_def(
        training_inputs, signature_outputs,
        REGRESS_METHOD_NAME)

    serving_signature_def = build_signature_def(
        serving_inputs, signature_outputs, PREDICT_METHOD_NAME
    )

    builder.add_meta_graph_and_variables(sess,
        [TRAINING, SERVING],
        signature_def_map={ REGRESS_METHOD_NAME: training_signature_def},
        strip_default_attrs=True)

    # builder.add_meta_graph(sess,
    #     [SERVING],
    #     signature_def_map={ PREDICT_METHOD_NAME: serving_inputs },
    #     strip_default_attrs=True)
    builder.save(as_text=False)

def train(sess):
    saver_dir = model_dir + "champion/checkpoints/"
    saver_prefix = saver_dir + "model"
    latest_checkpoint = tf.train.latest_checkpoint(saver_dir)
    print(latest_checkpoint)
    saver = tf.train.import_meta_graph(latest_checkpoint + ".meta")
    saver.restore(sess, latest_checkpoint)
    print("{}".format(latest_checkpoint))
    example_ph = tf.get_collection('example')[0]
    label_ph = tf.get_collection('label')[0]
    train_op = tf.get_collection('train_op')[0]
    global_step = tf.get_collection('global_step')[0]
    
    # for v in [n.name for n in tf.get_default_graph().as_graph_def().node]:
    #     print(v)
    # print("before train loop", sess.run(tf.report_uninitialized_variables()))
    epoch = 0
    with catch_sigint() as got_sigint:
        while True:
            if got_sigint():
                break

            dataset = make_dataset(minibatch_size, dataset_dir)
            iterator = dataset.make_initializable_iterator()
            example_it, label_it = iterator.get_next()
            minibatch = sess.run(global_step)
            for i in range(snapshot_epochs):
                sess.run(iterator.initializer)
                while True:
                    if got_sigint():
                        break
                    try:
                        e = sess.run(example_it)
                        l = sess.run(label_it)
                        sess.run(train_op, feed_dict={example_ph: e, label_ph: l})

                    except tf.errors.OutOfRangeError:
                        break
                    if minibatch % 1000 == 0:
                        saver.save(sess, saver_prefix, global_step)
                epoch += 1

            print("EPOCH {} FINISHED ({} minibatches of {} examples)".format(epoch, minibatch, minibatch_size))
            save_savedmodel(sess, model_dir + "champion/saved_model")                    
            take_snapshot(sess, saver, global_step)

def init_model(sess):
    # add tensors (and corresponding ops) to the default graph
    # we have to add the an iterator handle to the graph so that training can feed the net by handle on
    # every step...I could not find a more efficient way to do this, see
    # https://github.com/tensorflow/tensorflow/issues/20098

    example = tf.placeholder(tf.uint8, name='example', shape=(None, 19))
    label = tf.placeholder(tf.float32, name='label', shape=(None, 9))

    tf.add_to_collection("example", example)
    tf.add_to_collection("label", label)

    dense = tf.layers.dense(tf.cast(example, tf.float32), units=64, activation=tf.nn.relu, name="dense")
    logits = tf.layers.dense(dense, units=9, activation=tf.nn.relu, name="logits")
    softmax = tf.nn.softmax(logits, name='softmax')
    tf.add_to_collection('softmax', softmax)
    global_step = tf.Variable(0, name='global_step', trainable=False)
    tf.add_to_collection('global_step', global_step)

    loss = tf.losses.mean_squared_error(labels=label, predictions=softmax)
    optimizer = tf.train.GradientDescentOptimizer(.01)

    train = optimizer.minimize(loss, name='train', global_step=global_step)
    tf.add_to_collection('train', train)
    saver = tf.train.Saver()

    # initialize a Session so we can run initializers. 
    # this will seed the model with random weights
    init = tf.group(
        tf.global_variables_initializer(), 
        tf.local_variables_initializer())

    sess.run(init)

    # collections = [ops.GraphKeys.VARIABLES + ops.GraphKeys.TRAINABLE_VARIABLES]
    # for key in collections:
    #     graph.add_to_collection(key, self)
    take_snapshot(sess, saver, global_step, "champion")
    take_snapshot(sess, saver, global_step)


class catch_sigint(object):
    def __init__(self):
        self.caught_sigint = False
    def note_sigint(self, signum, frame):
        self.caught_sigint = True
    def __enter__(self):
        self.oldsigint = signal.signal(signal.SIGINT, self.note_sigint)
        return self
    def __exit__(self, *args):
        signal.signal(signal.SIGINT, self.oldsigint)
    def __call__(self):
        return self.caught_sigint

with tf.Session() as sess:
    if args.init:
        init_model(sess)
    else:
        train(sess)
