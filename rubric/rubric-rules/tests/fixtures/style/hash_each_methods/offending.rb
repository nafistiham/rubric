hash.keys.each { |k| puts k }
options.values.each do |v|
  process(v)
end
