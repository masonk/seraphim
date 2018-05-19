import tensorflow as tf

golden = "src/io/testing/golden.tfrecord"
writer = tf.python_io.TFRecordWriter(golden)

writer.write("The Quick Brown Fox".encode("utf-8"))
