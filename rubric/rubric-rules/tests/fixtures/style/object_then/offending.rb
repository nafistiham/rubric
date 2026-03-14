foo.yield_self { |x| x.to_s }
value.yield_self(&method(:process))
result = obj.yield_self { |v| v * 2 }
