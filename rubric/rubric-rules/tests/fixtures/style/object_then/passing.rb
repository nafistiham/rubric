foo.then { |x| x.to_s }
value.then(&method(:process))
# yield_self is mentioned in a comment only
name = "yield_self_method"
