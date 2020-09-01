#!/usr/bin/env ruby

require 'socket'

port = (ENV['PORT'] or 57482).to_i
server = TCPServer.new("127.0.0.1", port)
puts "Listening at http://127.0.0.1:#{port}"

while connection = server.accept
  request = connection.gets

  header_chunk, body = request.partition("\r\n\r\n")
  route, header_lines = header_chunk.partition("\r\n")
  headers = header_lines.split("\r\n")

  case route
  when /^OPTIONS .*$/
    connection.print "HTTP/1.1 204 No Content\r\n"
  when /^PUT \/skus\.json HTTP\/1.1$/
    File.write("./stadia.st/-/skus.json", body)
    connection.print "HTTP/1.1 204 No Content\r\n"
  when /^PUT \/index\.html HTTP\/1.1$/
    File.write("./stadia.run/index.html", body)
    connection.print "HTTP/1.1 204 No Content\r\n"
  else
    connection.print "HTTP/1.1 404 Not Found\r\n"
  end

  connection.print "Access-Control-Allow-Origin: *\r\n"
  connection.print "Access-Control-Allow-Methods: *\r\n"
  connection.print "\r\n"

  connection.close
end
