var ime = getUsername(), timestamp = 0;
$(document).ready(function () {
    $.ajax(
        {
            url: "./app/jesamPrvi.php",
            type: "GET",
            dataType: "json",
            async: false,
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

    $( "#btn" ).on( "click", posaljiPoruku );

    cekajPoruku();
})

function waitConfirmation(){
    while(!confirm("Press okay to start game!")){
    }
    $.ajax( 
        {
            url: "./app/notifyAll.php",
            type: "GET",
            async: false,
            success: function( data )
            {
                console.log("Svi su obavijesteni")
            },
            error: function( xhr, status ) 
            {
                console.log(xhr);
                if( status !== null )
                    console.log( "waitConfirmation :: greška pri slanju poruke (" + status + ")" );
            }
        } );
    waitOnHost();
}

function waitOnHost(){
    $.ajax(
        {
            url: "./app/hostReady.php",
            type: "GET",
            async: false,
            success: function( data ) 
            {
                // Sljedeća naredba ne treba: kako je dataType="json", data je već konvertiran iz stringa u objekt.
                // var data = JSON.parse( data );
                console.log( "HOST je ready" + JSON.stringify( data ) );
                setTimeout(() => {
                    $("#btn").prop('disabled', true);
                }, 60000);
            },
            error: function( xhr, status )
            {
                console.log( "cekajPoruku :: error :: status = " + status );
                // Nešto je pošlo po krivu...
                // Ako se dogodio timeout, tj. server nije ništa poslao u zadnjih XY sekundi,
                // pozovi ponovno cekajPoruku.
                if( status === "timeout" )
                    waitOnHost();
            }
        } );
}

function cekajPoruku() 
{
    console.log( "cekajPoruku" );

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
    console.log( "posaljiPoruku" );
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