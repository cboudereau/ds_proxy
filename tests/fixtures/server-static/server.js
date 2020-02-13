var http = require('http');
var fs = require('fs');

var app = http.createServer(function (req, res) {
  req.pipe(fs.createWriteStream(__dirname + '/uploads/' +req.url));

  res.writeHead(200, {'Content-Type': 'text/plain'});
  res.end('OK!');
});

app.listen(3000);