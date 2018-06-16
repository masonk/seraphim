from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

from pathlib import Path
import os
import tensorflow as tf
import argparse
import os

parser = argparse.ArgumentParser(description='Initialize a TicTacToe expert model.')
parser.add_argument('name', metavar='foo-model', help='Model prefix')
args = parser.parse_args()

model_dir = "src/tictactoe/saved_models/" + args.name 
latest_checkpoint = tf.train.latest_checkpoint(model_dir)
meta_graph = ".".join([latest_checkpoint, "meta"])

with tf.Session() as sess:
    new_saver = tf.train.import_meta_graph(meta_graph)
    print("{}".format(meta_graph))
    new_saver.restore(sess, latest_checkpoint)
    print("{}".format(latest_checkpoint))
    train_op = tf.get_collection('train_op')[0]
    game_feature = tf.get_collection('game_feature')[0]
