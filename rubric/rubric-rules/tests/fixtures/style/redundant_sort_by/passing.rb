arr.sort
arr.sort_by { |x| x.name }
arr.sort_by { |x| -x }
arr.sort_by(&:name)
items.sort_by { |x| x.length }
