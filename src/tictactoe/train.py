from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

from pathlib import Path
import argparse
import glob
import os
import tensorflow as tf

parser = argparse.ArgumentParser(description='Initialize a TicTacToe expert model.')
parser.add_argument('name', metavar='foo-model', help='Model prefix')
args = parser.parse_args()

model_dir = "src/tictactoe/saved_models/" + args.name 
saver_prefix = "src/tictactoe/saved_models/" + args.name + "/" + args.name

latest_checkpoint = tf.train.latest_checkpoint(model_dir)

num_epochs = 100
minibatch_size = 128
dataset_dir = "src/tictactoe/gamedata"
def make_dataset(minibatch_size, dataset_dir):
    files = glob.glob("{}/*.tfrecord".format(dataset_dir))
    print(files)
    dataset = tf.data.TFRecordDataset(files)
    dataset = dataset.map(parse)
    dataset = dataset.shuffle(buffer_size=100000)
    dataset = dataset.batch(minibatch_size)
    return dataset

def parse(bytes):
  features = {"game": tf.FixedLenFeature((), tf.string),
              "choice": tf.FixedLenSequenceFeature((), tf.float32, allow_missing=True)}
  parsed_features = tf.parse_single_example(bytes, features)
  game = tf.decode_raw(parsed_features["game"], tf.uint8)
  choice =  parsed_features["choice"]
  return tf.reshape(game, [19]), tf.reshape(choice, [9])

with tf.Session() as sess:
    dataset = make_dataset(minibatch_size, dataset_dir)
    iterator = dataset.make_initializable_iterator()

    saver = tf.train.import_meta_graph(latest_checkpoint + ".meta")
    saver.restore(sess, latest_checkpoint)
    print("{}".format(latest_checkpoint))
    
    training_handle = sess.run(iterator.string_handle())
    print(tf.get_collection('iterator_handle'))
    iterator_handle = tf.get_collection('iterator_handle')[0]

    train_op = tf.get_collection('train_op')[0]
    init = tf.get_collection('init')[0]
    global_step = tf.get_collection('global_step')[0]
    
    # for v in [n.name for n in tf.get_default_graph().as_graph_def().node]:
    #     print(v)
    # print("before train loop", sess.run(tf.report_uninitialized_variables()))

    for i in range(num_epochs):
        sess.run(iterator.initializer)
        while True:
            try:
                sess.run(train_op, feed_dict={iterator_handle: training_handle})
            except tf.errors.OutOfRangeError:
                break
            print(saver.save(sess, saver_prefix, global_step))

