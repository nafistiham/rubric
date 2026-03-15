array.map { |x| [x, x * 2] }.to_h
items.map { |item| [item.id, item] }.to_h
collection.map { |x| do_something(x) }.to_h
