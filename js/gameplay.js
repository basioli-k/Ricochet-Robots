var ime = getUsername(), timestamp = 0, countdown = 0, winner = "", faza_igre = 0;

$(document).ready(function () {
    $.ajax(
        {
            url: "./app/jesamPrvi.php",
            type: "GET",
            dataType: "json",
            // async: false,
            beforeSend: function(){
                console.log("prije ajaxa skrivam body");
                $('body').hide();
            },
            success: function( data ) 
            {
                if (data.prvi)
                    waitConfirmation()
                else
                    waitOnHost()
            },
            error: function( xhr, status )
            {
                console.log( "cekajPoruku :: error :: status = " + status );
            }
        } );

})

function postaviTimer(time) {
    $.ajax( 
        {
            url: "./app/postaviTimer.php",
            type: "GET",
            data: {
                wait: time,
            },
            // async: false,
            success: function( data )
            {
                console.log("Svi ce biti obavijesteni za: " + time + " sekundi");
            },
            error: function( xhr, status ) 
            {
                console.log(xhr);
                if( status !== null )
                    console.log( "FAIL (" + status + ")" );
            }
    });

}

function waitConfirmation(){
    while(!confirm("Press okay to start game!")){
    }
    postaviTimer(0);
    waitOnHost();
}

function allowRobotMovement(){
    var robot = null;
    $( ".robot_field" ).each(function(index) {
        $(this).on("click", () => {
            robot = $(this).css('background-color');
        });
    });
    document.onkeydown = function(event) {
        let direction = null;

        switch (event.key){
            case 'w':
                direction = "top";
                break;
            case 'a':
                direction = "left";
                break;
            case 's':
                direction = "bottom";
                break;
            case 'd':
                direction = "right";
                break;
            default:
                break;
        }

        if(robot != null && direction != null){
            var rgb = robot;
            rgb = rgb.replace(/[^\d,]/g, '').split(',');
            console.log(rgb);

            const hexColor = rgbToHex(rgb[0], rgb[1], rgb[2]);
            
            data = {
                username: getUsername(),
                robot: hexColor,
                direction: direction
            };

            move_robot(data);
        }


    };
    
}

function waitOnHost(){
    $.ajax(
        {
            url: "./app/cekajTimer.php",
            type: "GET",
            // async: false,
            success: function( data ) 
            {
                // Sljedeća naredba ne treba: kako je dataType="json", data je već konvertiran iz stringa u objekt.
                // var data = JSON.parse( data );
                console.log( "HOST je ready" + JSON.stringify( data ) );



                $('body').show();
                $( "#btn" ).on( "click", posaljiPoruku );

                cekajPoruku();

                igraj("licitacija");

                // igraj("licitacija");
                // TODO ovo ovdje je sada tu cisto da pokazujemo kako stvari idu
                // inace se ova funkcija poziva samo igracu koji trenutno pokazuje rjesenje

            },
            error: function( xhr, status )
            {
                console.log( "cekajPoruku :: error :: status = " + status );
                // Nešto je pošlo po krivu...
                // Ako se dogodio timeout, tj. server nije ništa poslao u zadnjih XY sekundi,
                // pozovi ponovno waitOnHost.
                if( status === "timeout" )
                    waitOnHost();
            }
        } );
}

function srediLicitaciju(licitacija) {
    console.log(licitacija);
    var glasovi = licitacija.split(',');
    var glasovi_sort = new Array(glasovi.length - 1);
    for (var i = 0; i < glasovi.length - 1; i++) {
        glasovi_sort[i] = glasovi[i].split(':'); 
    }
    
    // Ovu funkciju bi trebalo malo prilagoditi pravilima igre.
    glasovi_sort.sort(function(a, b) {
        var a2 = parseInt(a[2]); var a0 = parseInt(a[0]);
        var b2 = parseInt(b[2]); var b0 = parseInt(b[0]);
        if (a2 !== b2) return a2 - b2;
        return a0 - b0;
    })
    return glasovi_sort;
}

function dajLicitaciju() {
    var licitacija;
    $.ajax({
        url: './app/dajLicitaciju.php',
        type: "GET",
        // Asinkrono jer se radi o upitu za podatcima.
        async: false,
        success: function(data) {

            console.log(licitacija);
            licitacija = srediLicitaciju(data.licitacija);
        }
    })
    return licitacija;
}

function igraj(faza_igre) {

    if (faza_igre === "licitacija") {
        console.log("licitacija");
        postaviTimer(60);
        $.ajax({
            url: "./app/cekajTimer.php",
            type: "GET",
            success: function (data) {
                igraj("pomicanje");
            }
        })
    }
    else if (faza_igre === "pomicanje") {
        var licitacija = dajLicitaciju();
        // popis onih koji su vec pokusali napraviti svoj potez.
        var odigrali = [];
        console.log(licitacija[0][1]);
        igraj("licitacija");
    }
}

function cekajPoruku() 
{
    // Recimo, koristimo $.ajax (možemo i $.get).
    $.ajax(
    {
        url: "./app/cekajPoruku.php",
        type: "GET",
        data:
        { 
            // Timestamp = vrijeme kad smo zadnji put dobili poruke sa servera.
            timestamp: timestamp, 

            // cache = svaki put šaljemo i trenutno vrijeme tako da browser ne pročita iz 
            //         cache-a odgovor servera, nego ga zaista ide kontaktirati.
            // Da smo koristili post, ovo ne bi bilo potrebno. (POST se ne cache-ira.)
            cache: new Date().getTime()
        },
        dataType: "json",
        success: function( data ) 
        {
            // Sljedeća naredba ne treba: kako je dataType="json", data je već konvertiran iz stringa u objekt.
            // var data = JSON.parse( data );

            console.log( "cekajPoruku :: success :: data = " + JSON.stringify( data ) );

            // Da li je u poruci definirano svojstvo error?
            // Uoči: naša PHP aplikacija će dodavati svojstvo error ako detektira neku grešku.
            if( typeof( data.error ) !== "undefined" )
            {
                // Ipak je došlo do greške!
                console.log( "cekajPoruku :: success :: server javio grešku " + data.error );
            }
            else
            {
                // Ako nema greške, pročitaj poruku i dodaj ju u div.
                $("#chat").append( "<div>" + decodeURI( data.msg ) + "</div>" );
                timestamp = data.timestamp;
            
                var licitacija = srediLicitaciju(data.licitacija);
                $("#ranking").empty();
                for (var i = 0; i < licitacija.length; i++) {
                    $("#ranking").append('<div>' + '<b>' + licitacija[i][1] + '</b>: ' + licitacija[i][2] + '</div>' );
                }
                
                // Ova poruka je gotova, čekaj iduću.
                cekajPoruku();
            }
        },
        error: function( xhr, status )
        {
            console.log( "cekajPoruku :: error :: status = " + status );
            // Nešto je pošlo po krivu...
            // Ako se dogodio timeout, tj. server nije ništa poslao u zadnjih XY sekundi,
            // pozovi ponovno cekajPoruku.
            if( status === "timeout" )
                cekajPoruku();
        }
    } );
}


function posaljiPoruku() 
{
    // Za slanje poruke koristimo GET, poslat ćemo ime i poruku.
    // Recimo, koristimo $.ajax (možemo i $.get).
    $.ajax( 
    {
        url: "./app/saljiPoruku.php",
        type: "GET",
        data: 
        { 
            ime: ime, 
            msg: encodeURI( $( "#txt" ).val() ) 
        },
        dataType: "json",
        success: function( data )
        {
            console.log( "posaljiPoruku :: success :: data = " + JSON.stringify( data ) );
        },
        error: function( xhr, status ) 
        {
            if( status !== null )
                console.log( "posaljiPoruku :: greška pri slanju poruke (" + status + ")" );
        }
    } );

    // Obriši sadržaj text-boxa.
    $( "#txt" ).val( "" );
} 
