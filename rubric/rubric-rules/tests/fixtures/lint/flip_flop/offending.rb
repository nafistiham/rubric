lines.each do |line|
  if (line =~ /begin/)..(line =~ /end/)
    puts line
  end
end
