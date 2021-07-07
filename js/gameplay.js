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

function waitConfirmation(){
    $('body').hide();
    while(!confirm("Press okay to start game!")){
    }
    $('body').show();
    // $.ajax( 
    //     {
    //         url: "./app/notifiyAll.php",
    //         type: "GET",
    //         data: 
    //         { 
    //             username: getUsername(), 
    //             msg: encodeURI( "Done" ) 
    //         },
    //         dataType: "json",
    //         success: function( data )
    //         {
    //             console.log( "waitConfirmation :: success :: data = " + JSON.stringify( data ) );
    //         },
    //         error: function( xhr, status ) 
    //         {
    //             if( status !== null )
    //                 console.log( "waitConfirmation :: greška pri slanju poruke (" + status + ")" );
    //         }
    //     } );

    //korisnik vidi
}

function waitOnHost(){
    //ne vidi se

    // $.ajax(
    //     {
    //         url: "./app/hostReady.php",
    //         type: "GET",
    //         data:
    //         { 
    //             // Timestamp = vrijeme kad smo zadnji put dobili poruke sa servera.
    //             timestamp: timestamp, 
    
    //             // cache = svaki put šaljemo i trenutno vrijeme tako da browser ne pročita iz 
    //             //         cache-a odgovor servera, nego ga zaista ide kontaktirati.
    //             // Da smo koristili post, ovo ne bi bilo potrebno. (POST se ne cache-ira.)
    //             cache: new Date().getTime()
    //         },
    //         dataType: "json",
    //         success: function( data ) 
    //         {
    //             // Sljedeća naredba ne treba: kako je dataType="json", data je već konvertiran iz stringa u objekt.
    //             // var data = JSON.parse( data );

    //             console.log( "cekajPoruku :: success :: data = " + JSON.stringify( data ) );
    
    //             // Da li je u poruci definirano svojstvo error?
    //             // Uoči: naša PHP aplikacija će dodavati svojstvo error ako detektira neku grešku.
    //             if( typeof( data.error ) !== "undefined" )
    //             {
    //                 // Ipak je došlo do greške!
    //                 console.log( "cekajPoruku :: success :: server javio grešku " + data.error );
    //             }
    //             else
    //             {
    //                 // Ako nema greške, pročitaj poruku i dodaj ju u div.
    //                 $("#chat").append( "<div>" + decodeURI( data.msg ) + "</div>" );
    //                 timestamp = data.timestamp;
                
    //                 // Ova poruka je gotova, čekaj iduću.
    //                 cekajPoruku();
    //             }
    //         },
    //         error: function( xhr, status )
    //         {
    //             console.log( "cekajPoruku :: error :: status = " + status );
    //             // Nešto je pošlo po krivu...
    //             // Ako se dogodio timeout, tj. server nije ništa poslao u zadnjih XY sekundi,
    //             // pozovi ponovno cekajPoruku.
    //             if( status === "timeout" )
    //                 cekajPoruku();
    //         }
    //     } );

    //vidi se
}

$(document).ready(function () {
    $.ajax(
        {
            url: "./app/jesamPrvi.php",
            type: "GET",
            dataType: "json",
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
        } 
    );
    allowRobotMovement()
})