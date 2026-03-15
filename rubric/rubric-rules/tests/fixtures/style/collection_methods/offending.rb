# These use non-preferred collection method aliases
result = [1, 2, 3].collect { |x| x * 2 }
found = [1, 2, 3].detect { |x| x > 1 }
all = [1, 2, 3].find_all { |x| x > 0 }
sum = [1, 2, 3].inject(0) { |acc, x| acc + x }
flat = [[1, 2], [3, 4]].collect_concat { |a| a }
