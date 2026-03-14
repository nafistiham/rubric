[1, 2, 3].each_with_object([]) { |x, memo| memo << x }
[1, 2, 3].each_with_object({}) { |x, memo| memo[x] = x }
items.each_with_object(MyClass.new) { |x, m| m.add(x) }
