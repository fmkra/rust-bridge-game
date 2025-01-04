$(function() {
    const socket = io('http://localhost:3000');
  
    socket.on('string to client', (data) => {
        console.log('string to client', data);
    });

    socket.on('struct to client', (data) => {
        console.log('struct to client', data);
    });

    $('#send').click(() => {
        const message = $('#input').val();
        console.log('sending', message);
        socket.emit('string to server', message);
    });

    $('#send_struct').click(() => {
        // const message = $('#input').val();
        // console.log('sending', message);
        socket.emit('struct to server', {
            field1: 'value1',
            field2: 123,
        });
    });
});