arr.sort
arr.sort { |a, b| b <=> a }
arr.sort_by { |a| a.name }
items.sort_by(&:name)
