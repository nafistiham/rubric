return if foo.nil? || foo.empty?
return if foo.blank?
# foo&.empty? is documented here as an anti-pattern
msg = "avoid foo&.empty? usage"
