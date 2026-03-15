array.select { |x| !x.nil? }
array.reject { |x| x.nil? }
array.filter { |x| !x.nil? }
items.select { |item| item != nil }
items.reject { |item| item == nil }
