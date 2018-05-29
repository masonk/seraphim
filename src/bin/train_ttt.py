import tensorflow as tf
import glob

num_epochs = 100
minibatch_size = 128
dataset_dir = "src/tictactoe/gamedata"
model_dir = "src/tictactoe/simple_model/checkpoint"
graph_filename = "src/tictactoe/simple_net.pb"

def make_dataset(minibatch_size, dataset_dir):
    files = glob.glob("{}/*.tfrecord".format(dataset_dir))
    print("loading", files)
    dataset = tf.data.TFRecordDataset(files)
    dataset = dataset.map(parse)
    dataset = dataset.shuffle(buffer_size=100000)
    dataset = dataset.batch(minibatch_size)
    print("loaded data")
    return dataset

def parse(bytes):
  features = {"game": tf.FixedLenFeature((), tf.string),
              "choice": tf.FixedLenSequenceFeature((), tf.float32, allow_missing=True)}
  parsed_features = tf.parse_single_example(bytes, features)
  game = tf.decode_raw(parsed_features["game"], tf.uint8)
  choice =  parsed_features["choice"]
  return tf.reshape(game, [19]), tf.reshape(choice, [9])


with tf.gfile.FastGFile(graph_filename,'rb') as f:
    sess = tf.InteractiveSession()

    dataset = make_dataset(minibatch_size, dataset_dir)
    print("loading graph at '{}'".format(graph_filename))

    iterator = dataset.make_initializable_iterator()
    example, label = iterator.get_next()
    graph_def = tf.GraphDef()
    graph_def.ParseFromString(f.read())
    tf.import_graph_def(graph_def, name='',input_map={'x': example, 'y_true':label})

    init = tf.group(
        tf.global_variables_initializer(), 
        tf.local_variables_initializer(), 
        iterator.initializer, 
        sess.graph.get_operation_by_name('init'))

    train = sess.graph.get_operation_by_name('train')
    # for name in [n.name for n in tf.get_default_graph().as_graph_def().node]:
    #     print(name)
    
    saver = tf.train.Saver()

    sess.run(init)

    for i in range(num_epochs):
        sess.run(iterator.initializer)
        
        while True:
            try:
                sess.run(train)
            except tf.errors.OutOfRangeError:
                break
        save_path = saver.save(sess, model_dir)
        print("Model saved in path: %s" % save_path)
