[1, 2, 3].each do |x|
  if x > 1
    puts x
  end
end

items.each do |item|
  if item.active?
    process(item)
  end
end
