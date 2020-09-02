#!/usr/bin/env ruby

require 'socket'

host = (ENV['HOST'] or "127.0.0.1").to_s
port = (ENV['PORT'] or 57482).to_i
server = TCPServer.new(host, port)
puts "Listening at http://#{host}:#{port}"

while connection = server.accept
  request = connection.gets

  header_chunk, body = request.partition("\r\n\r\n")
  route_line, header_lines = header_chunk.partition("\r\n")
  headers = header_lines.split("\r\n")
  route = route_line.sub(/[ ]+HTTP\/1\.[01]$/, '')

  client = connection.addr[2]

  response = nil
  status = case route
    when /^GET \/skus\.json$/
      response = File.read("./stadia.st/-/skus.json", :encoding => "utf-8")
      "200 OK"
    when /^PUT \/skus\.json$/
      File.write("./stadia.st/-/skus.json", body)
      "204 No Content"

    when /^GET \/index\.html$/
      response = File.read("./stadia.run/index.html", :encoding => "utf-8")
      "200 OK"
    when /^PUT \/index\.html$/
      File.write("./stadia.run/index.html", body)
      "204 No Content"

    when /^OPTIONS \/.+$/
      "204 No Content"
    else
      "404 Not Found"
  end

  puts "#{client}  |  #{route}  |  #{status}"

  connection.print "HTTP/1.1 #{status}\r\n"
  connection.print "Access-Control-Allow-Origin: *\r\n"
  connection.print "Access-Control-Allow-Methods: *\r\n"

  unless response.nil?
    connection.print "Content-Type: text/plain; charset=utf-8\r\n"
    connection.print "Content-Length: #{response.bytesize}\r\n"
    connection.print "\r\n"
    connection.print response
  end

  connection.close
end
