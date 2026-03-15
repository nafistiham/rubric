# Regex character ranges that mix upper and lower case

str.match?(/[A-z]/)
str.gsub(/[A-z]/, '')
str =~ /[a-Z]/
/[Z-a]/.match(str)
str.scan(/[B-y]/)
