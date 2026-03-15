[1, 2, 3].each do |x|
  next unless x > 1
  puts x
end

items.each do |item|
  next unless item.active?
  process(item)
end

[1, 2, 3].each do |x|
  if x > 1
    puts x
  else
    puts "small"
  end
end
