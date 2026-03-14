hash.each_key { |k| puts k }
options.each_value do |v|
  process(v)
end
