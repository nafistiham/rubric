[1, 2, 3].each_with_object(0) { |x, memo| memo + x }
[:a, :b].each_with_object(:init) { |x, memo| x }
items.each_with_object(true) { |x, m| m }
items.each_with_object(false) { |x, m| m }
items.each_with_object(nil) { |x, m| m }
