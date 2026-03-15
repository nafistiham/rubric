array.to_h { |x| [x, x * 2] }
items.to_h { |item| [item.id, item] }
collection.to_h { |x| do_something(x) }
