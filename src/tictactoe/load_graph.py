
import tensorflow as tf
import os
import numpy as np
from tensorflow.python.platform import gfile

train =  tf.data.Dataset.from_tensor_slices({
    state: tf.placeholder(dtype=tf.uint8, shape = [1,19]),
    action: tf.placeholder(dtype=tf.f32, shape=[1,9])
})
with tf.Session() as sess: 
  print("load graph")
  with gfile.FastGFile("src/tictcatoe/simple_net.pb",'rb') as f:
    graph_def = tf.GraphDef()
    graph_def.ParseFromString(f.read())
    sess.graph.as_default()
    tf.import_graph_def(graph_def, name='')
  print("map variables")
  persisted_result = sess.graph.get_tensor_by_name("saved_result:0")
  tf.add_to_collection(tf.GraphKeys.VARIABLES,persisted_result)
  try:
    saver = tf.train.Saver(tf.all_variables()) # 'Saver' misnomer! Better: Persister!
  except:pass
  print("load data")
  saver.restore(persisted_sess, "checkpoint.data")  # now OK
  print(persisted_result.eval())
  print("DONE")