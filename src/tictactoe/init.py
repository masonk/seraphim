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

model_dir = "src/tictactoe/saved_models/" + args.name + "/" + args.name

with tf.Session() as sess:
    # The input is the state of a Tic Tac Toe game.
    # This is represented as two length-9 Vec<u8>.
    # The first plane holds the location of the first player's stones,
    # The second plane, the second player's.
    # A 19th byte holds 0 for first player, 1 for second player.
    game_feature = tf.placeholder(tf.uint8, shape=[None, 9 * 2 + 1], name ='game_feature')
    tf.add_to_collection('game_feature', game_feature)

    # Training makes makes the net more likely to pick the picked move.
    # The picked move will be 1.0, the other 8 spaces will be 0.0.
    action_label = tf.placeholder(tf.float32, shape=[None, 9], name='action_label')
    tf.add_to_collection('action_label', action_label)

    dense = tf.layers.dense(tf.cast(game_feature, tf.float32), units=64, activation=tf.nn.relu)
    logits = tf.layers.dense(dense, units=9, activation=tf.nn.relu)
    softmax = tf.nn.softmax(logits, name='softmax')
    tf.add_to_collection('softmax', softmax)

    sess = tf.Session()
    init = tf.variables_initializer(tf.global_variables(), name='init')
    sess.run(init)

    loss = tf.losses.mean_squared_error(labels=action_label, predictions=softmax)
    optimizer = tf.train.GradientDescentOptimizer(.01)
    train = optimizer.minimize(loss, name='train')
    tf.add_to_collection('train', train)

    saver = tf.train.Saver()
    saved = saver.save(sess, model_dir, global_step=0)
    print("Model saved in path: %s" % saved)
