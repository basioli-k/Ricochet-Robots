function getUsername(){
    var cookie = document.cookie.split(";").map( c => c.split("=")).reduce((acc, c) => {
        acc[decodeURIComponent(c[0].trim())] = decodeURIComponent(c[1].trim());
        return acc;
    }, {});

    return cookie.username
}

function move_robot(data){
  $.ajax( 
  {
      url: "./app/moveRobot.php",
      type: "POST",
      data: data,
      dataType: "json",
      success: function( data )
      {
          console.log( "USPJEH " + JSON.stringify( data ) );
          location.reload();

      },
      error: function( xhr, status ) 
      {
          console.log(xhr);
          if( status !== null )
              console.log( "FAIL (" + status + ")" );
      }
  } );
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