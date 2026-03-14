items.select { |item| item =~ /foo/ }
names.select { |n| n.match?(/^foo/) }
items.reject { |item| item =~ /bar/ }
