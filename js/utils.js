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

function get_new_token( round ) {
  $.ajax({
    url: './app/dajLog.php',
    data: {
        filename: 'tokens.log',
        round: round
    },
    type: "GET",
    // Asinkrono jer se radi o upitu za podatcima.
    async: false,
    success: function(data) {
        if( typeof( data.error ) !== "undefined" ) 
            console.log( "dajPoteze :: success :: server javio grešku " + data.error );
        if (data.game_over){
          cilj = null;
        }
        else{
          cilj.znak = data.tokens.split(":")[1];
          cilj.boja = data.tokens.split(":")[0];
        }
        
    },
    error: function (xhr, status) {
        console.log(xhr);
    }
  })

}

function draw_goal(){

  $('#token').html("<i class=\"" + cilj.znak + "\" style = \"color:" + cilj.boja + "\"></i>"); 
  // $('#token').css("font-size", "xx-large")
}