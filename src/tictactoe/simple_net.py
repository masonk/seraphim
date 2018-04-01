from __future__ import absolute_import
from __future__ import division
from __future__ import print_function

from pathlib import Path
import tensorflow as tf

print(Path(__file__).with_suffix("pb"))

definition = tf.Session().graph_def
directory = 'src/tictactoe'
tf.train.write_graph(definition, directory, Path(__file__).with_suffix("pb"), as_text=False)