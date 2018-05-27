import tensorflow as tf
import third_party.tensorflow.core.example.example_pb2 as ex
def parse(bytes):
  
  features = {"game": tf.FixedLenFeature((), tf.string),
              "choice": tf.FixedLenFeature((), tf.float32)}
  parsed_features = tf.parse_single_example(bytes, features)
  return parsed_features["game"], parsed_features["choice"]

with tf.Session() as sess:
    with  open("examples.pb", "rb") as f:  
        data = f.read(87)
        e = ex.Example()
        e.ParseFromString(data)
        print(data)
        print(e)

        game,choice = parse(data)
        tf.cast(choice, tf.string)
        a = tf.Print(choice, [choice], message="This is a: ", summarize=50)

        a.eval()


with  open("examples.pb", "rb") as f:
    e = ex.Example()
    e.ParseFromString(data)
    print(data)
    print(e)


