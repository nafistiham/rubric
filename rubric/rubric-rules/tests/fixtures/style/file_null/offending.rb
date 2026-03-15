system("command > /dev/null")
File.open("/dev/null", "w")
redirect_output('/dev/null')
