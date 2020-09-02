#!/usr/bin/env ruby
require 'socket'

host = (ENV['HOST'] or "127.0.0.1").to_s
port = (ENV['PORT'] or 57482).to_i

server = TCPServer.new(host, port)
puts "Listening at http://#{host}:#{port}"

while connection = server.accept
  client_address = connection.addr[2]

  header_chunk = connection.gets("\r\n\r\n")

  route_line, _, header_chunk = header_chunk.partition(/\r\n/)
  headers = Hash[ header_chunk.split(/\r\n/).collect { |line|
    key, _, value = line.partition(/[ ]*:[ ]*/)
    [key.downcase.gsub(/\-/, '_').to_sym, value]
  } ]
  route = route_line.sub(/[ ]+HTTP\/1\.[01]$/, '')
  method, _, path = route.partition(/[ ]+/)

  content_length = headers[:content_length]
  body = unless content_length.nil?
    connection.gets(nil, content_length.to_i)
  end

  response = nil
  status = case route
    when /^GET\s+\/skus\.json$/
      response = File.read("./stadia.st/-/skus.json", :encoding => "utf-8")
      "200 OK"
    when /^PUT\s+\/skus\.json$/
      File.write("./stadia.st/-/skus.json", body, :encoding => "utf-8")
      "204 No Content"

    when /^GET\s+\/index\.html$/
      response = File.read("./stadia.run/index.html", :encoding => "utf-8")
      "200 OK"
    when /^PUT\s+\/index\.html$/
      File.write("./stadia.run/index.html", body, :encoding => "utf-8")
      "204 No Content"

    when /^OPTIONS\s+\/.+$/
      "204 No Content"
    else
      "404 Not Found"
  end

  puts "#{client_address}  |  #{route}  |  #{status}"

  connection.print "HTTP/1.1 #{status}\r\n"

  unless headers[:origin].nil? or headers[:access_control_request_method].nil?
    connection.print "Access-Control-Allow-Origin: #{
      headers[:origin]}\r\n"
    connection.print "Access-Control-Allow-Methods: #{
      headers[:access_control_request_method]}\r\n"
  end

  unless response.nil?
    connection.print "Content-Type: text/plain; charset=utf-8\r\n"
    connection.print "Content-Length: #{response.bytesize}\r\n"
    connection.print "\r\n"
    connection.print response
  end

  connection.close
end
