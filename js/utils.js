function getUsername(){
    var cookie = document.cookie.split(";").map( c => c.split("=")).reduce((acc, c) => {
        acc[decodeURIComponent(c[0].trim())] = decodeURIComponent(c[1].trim());
        return acc;
    }, {});

    return cookie.username
}

function componentToHex(c) {
    var hex = parseInt(c).toString(16);
    return hex.length == 1 ? "0" + hex : hex;
  }
  
  function rgbToHex(r, g, b) {
    return "#" + componentToHex(r) + componentToHex(g) + componentToHex(b);
  }
  

// $(document).ready(function () {
//     console.log(getUsername())
// })