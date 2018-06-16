import tensorflow as tf
import glob



parser = argparse.ArgumentParser(description='Load and train a TicTacToe expert model.')
parser.add_argument('name', metavar='foo-model', help='Model prefix')
args = parser.parse_args()

num_epochs = 100
minibatch_size = 128
checkpoint = "src/tictactoe/saved_models/" + args.name + "/checkpoint"

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

with tf.Session() as sess:
  # Restore variables from disk.
  saver.restore(sess, checkpoint)
  print("Model restored.")
  # Check the values of the variables
  print("v1 : %s" % v1.eval())
  print("v2 : %s" % v2.eval())