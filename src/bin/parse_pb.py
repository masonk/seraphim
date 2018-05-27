import tensorflow as tf
import third_party.tensorflow.core.example.example_pb2 as ex
def parse(bytes):
  
  features = {"game": tf.FixedLenFeature((), tf.string),
              "choice": tf.FixedLenSequenceFeature((), tf.float32, allow_missing=True)}
  parsed_features = tf.parse_single_example(bytes, features)
  return tf.decode_raw(parsed_features["game"], tf.uint8), parsed_features["choice"]

with tf.Session() as sess:
    with  open("examples.pb", "rb") as f:  
        data = f.read(87)
        e = ex.Example()
        e.ParseFromString(data)
        print(data)
        print(e)

        game,choice = parse(data)
        tf.cast(choice, tf.string)
        a = tf.Print(choice, [game, choice], message="This is a: ", summarize=50)

        a.eval()


with  open("examples.pb", "rb") as f:
    e = ex.Example()
    e.ParseFromString(data)
    print(data)
    print(e)


