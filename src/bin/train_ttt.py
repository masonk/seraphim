import tensorflow as tf
import glob

def parse(bytes):
  
  features = {"game": tf.FixedLenFeature((), tf.string),
              "choice": tf.FixedLenSequenceFeature((), tf.float32, allow_missing=True)}
  parsed_features = tf.parse_single_example(bytes, features)
  game = tf.decode_raw(parsed_features["game"], tf.uint8)
  choice =  parsed_features["choice"]
  return tf.reshape(game, [19]), tf.reshape(choice, [9])

num_epochs = 1
dataset_dir = "src/tictactoe/gamedata"
files = glob.glob("{}/*".format(dataset_dir))
print("loading", files)
dataset = tf.data.TFRecordDataset(files)
dataset = dataset.map(parse)
dataset = dataset.shuffle(buffer_size=100000)
dataset = dataset.batch(32)
print("loaded data")

graph_filename = "src/tictactoe/simple_net.pb"
print("loading graph at '{}'".format(graph_filename))

with tf.Session() as sess:
    iterator = dataset.make_initializable_iterator()
    example, label = iterator.get_next()

    with tf.gfile.FastGFile(graph_filename,'rb') as f:
        graph_def = tf.GraphDef()
        graph_def.ParseFromString(f.read())
        sess.graph.as_default()
        tf.import_graph_def(graph_def, name='',input_map={'x': example, 'y_true':label})

    init = tf.group(
        tf.global_variables_initializer(), 
        tf.local_variables_initializer(), 
        iterator.initializer, 
        sess.graph.get_operation_by_name('init'))

    sess.run(init)

    train = sess.graph.get_operation_by_name('train')

    # print(dataset.output_types) 
    # print(dataset.output_shapes)

    while True:
        for _ in range(num_epochs):
            sess.run(iterator.initializer)
            try:
                sess.run(train)
            except tf.errors.OutOfRangeError:
                break
