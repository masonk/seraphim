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
# This is represented as two length-9 boolean vectors 
# the location of the first player's stones in the first, and the second player's stones
# in the second, and whose move it is in a final bool - 0 for first player, 1 for second player.
x = tf.placeholder(tf.bool, shape=[1, 9 * 2 + 1], name ='x')
y_true = tf.placeholder(tf.half, shape=[1, 9], name='y_true')

dense = tf.layers.dense(tf.cast(x, tf.half), units=64, activation=tf.nn.relu)
logits = tf.layers.dense(dense, units=9, activation=tf.nn.relu)
softmax = tf.nn.softmax(logits)

sess = tf.Session()
init = tf.variables_initializer(tf.global_variables(), name='init')
sess.run(init)

loss = tf.losses.mean_squared_error(labels=y_true, predictions=softmax)
optimizer = tf.train.GradientDescentOptimizer(.01)
train = optimizer.minimize(loss, name='train')

definition = tf.Session().graph_def
tf.train.write_graph(definition, outdir, outfile, as_text=False)