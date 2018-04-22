from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

from pathlib import Path
import os
import tensorflow as tf

outdir = os.path.dirname(__file__)
outfile = Path(__file__).stem + ".pb"

print(os.path.join(outdir, outfile))

# The input is the state of a Tic Tac Toe game.
# This is represented as two length-9 Vec<u8>.
# The first plane holds the location of the first player's stones,
# The second plane, the second player's.
# A 19th byte holds 0 for first player, 1 for second player.
x = tf.placeholder(tf.uint8, shape=[1, 9 * 2 + 1], name ='x')

# Training makes makes the net more likely to pick the picked move.
# The picked move will be 1.0, the other 8 spaces will be 0.0.
y_true = tf.placeholder(tf.float32, shape=[1, 9], name='y_true')

dense = tf.layers.dense(tf.cast(x, tf.float32), units=64, activation=tf.nn.relu)
logits = tf.layers.dense(dense, units=9, activation=tf.nn.relu)
softmax = tf.nn.softmax(logits, name='softmax')

sess = tf.Session()
init = tf.variables_initializer(tf.global_variables(), name='init')
sess.run(init)

loss = tf.losses.mean_squared_error(labels=y_true, predictions=softmax)
optimizer = tf.train.GradientDescentOptimizer(.01)
train = optimizer.minimize(loss, name='train')
saver = tf.train.Saver(tf.global_variables())

definition = tf.Session().graph_def
tf.train.write_graph(definition, outdir, outfile, as_text=False)