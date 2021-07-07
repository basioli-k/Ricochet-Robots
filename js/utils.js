function getUsername(){
    var cookie = document.cookie.split(";").map( c => c.split("=")).reduce((acc, c) => {
        acc[decodeURIComponent(c[0].trim())] = decodeURIComponent(c[1].trim());
        return acc;
    }, {});

    return cookie.username
}


// $(document).ready(function () {
//     console.log(getUsername())
// })