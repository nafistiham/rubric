# These use preferred collection methods
result = [1, 2, 3].map { |x| x * 2 }
found = [1, 2, 3].find { |x| x > 1 }
all = [1, 2, 3].select { |x| x > 0 }
sum = [1, 2, 3].reduce(0) { |acc, x| acc + x }
flat = [[1, 2], [3, 4]].flat_map { |a| a }

# Comments mentioning collect, detect, inject should not be flagged
# arr.collect is an alias for arr.map
x = obj.collector  # not a flagged method (collector != collect)
y = obj.injected   # not a flagged method (injected != inject)
