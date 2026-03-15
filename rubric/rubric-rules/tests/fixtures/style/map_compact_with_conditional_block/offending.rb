[1, nil, 2].map { |x| x * 2 if x }.compact
items.map { |x| x.name if x.active? }.compact
array.map { |x| x > 0 ? x : nil }.compact
