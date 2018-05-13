import tensorflow as tf
import glob

num_epochs = 10
dataset_dir = "src/tictactoe/gamedata"
files = glob.glob("{}/*".format(dataset_dir))
print("loading {}", files)
dataset = tf.data.TFRecordDataset(files)
dataset = dataset.map(_parse_function)
dataset = dataset.shuffle(buffer_size=100000)
dataset = dataset.batch(32)
print("loaded data")

graph_filename = src/tictactoe/simple_net.pb"
print("loading graph at '{}'".format(graph_filename))
with gfile.FastGFile("src/tictcatoe/simple_net.pb",'rb') as f:
    graph_def = tf.GraphDef()
    graph_def.ParseFromString(f.read())
    sess.graph.as_default()
    tf.import_graph_def(graph_def, name='')

train = sess.graph.get_tensor_by_name('train')
example_ph = sess.graph.get_tensor_by_name('x')
label_ph = sess.graph.get_tensor_by_name('y_true')
iterator = dataset.make_initializable_iterator()
next_element = iterator.get_next()

sess.run(iterator.intializer)
while True:
    for _ in range(num_epochs):
        sess.run(iterator.initializer)
        try:
            example, label = sess.run(next_element)

            sess.run(train)
        except tf.errors.OutOfRangeError:
            break


def _parse_function(example_proto):
  features = {"game": tf.FixedLenFeature((), tf.int64),
              "choice": tf.FixedLenFeature((), tf.float32, default_value=0.0)}
  parsed_features = tf.parse_single_example(example_proto, features)
  return parsed_features["game"], parsed_features["choice"]
