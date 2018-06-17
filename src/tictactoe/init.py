from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

from pathlib import Path

import argparse
import os
import tensorflow as tf

parser = argparse.ArgumentParser(description='Initialize a TicTacToe expert model.')
parser.add_argument('name', metavar='foo-model', help='Model prefix')
args = parser.parse_args()
model_dir = "src/tictactoe/saved_models/" + args.name + "/" + args.name

with tf.Session() as sess:
    iterator_handle = tf.placeholder(tf.string, shape=[])
    tf.add_to_collection('iterator_handle', iterator_handle)

    iterator = tf.data.Iterator.from_string_handle(iterator_handle, (tf.uint8, tf.float32), ((None, 19), (None, 9)))
    example, label = iterator.get_next()

    dense = tf.layers.dense(tf.cast(example, tf.float32), units=64, activation=tf.nn.relu)
    logits = tf.layers.dense(dense, units=9, activation=tf.nn.relu)
    softmax = tf.nn.softmax(logits, name='softmax')
    tf.add_to_collection('softmax', softmax)
    global_step = tf.Variable(0, name='global_step', trainable=False)
    tf.add_to_collection('global_step', global_step)
    
    sess = tf.Session()
    init = tf.group(
        tf.global_variables_initializer(), 
        tf.local_variables_initializer())
    sess.run(init)
    tf.add_to_collection('init', init)
    loss = tf.losses.mean_squared_error(labels=label, predictions=softmax)
    optimizer = tf.train.GradientDescentOptimizer(.01)

    train = optimizer.minimize(loss, name='train', global_step=global_step)
    tf.add_to_collection('train', train)

    saver = tf.train.Saver()
    saved = saver.save(sess, model_dir, global_step=global_step)
    print("Model saved in path: %s" % saved)
