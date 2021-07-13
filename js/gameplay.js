var ime = getUsername(), timestampPoruka = 0, timestampPotez = 0;
var countdown = 0, winner = "", povuceniPotezi = 0, odigrali = [], vrijemeZaLicitaciju = 20;
var cilj = {
    znak: null, //"fas fa-star",  OSTAVLJAM TI ZAKOMENTIRANO DA VIDIS KAKO CE IZGLEDAT, OCEKUJEM fas razmak znak
    boja: null//"#0000ff",
};

var pozicijeRobota = [];

$(document).ready(function () {
    $.ajax(
        {
            url: "./app/jesamPrvi.php",
            type: "GET",
            dataType: "json",
            // async: false,
            beforeSend: function(){
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
                    console.log( "FAIL postaviTimer (" + status + ")" );
            }
    });

}

function waitConfirmation(){
    while(!confirm("Press okay to start game!")){
    }
    waitOnHost();
    postaviTimer(0);
}

function allowRobotMovement(brojPoteza){
    var rbt = null;
    $( ".robot_field" ).each(function(index) {
        $(this).on("click", () => {
            rbt = $(this).css('background-color');
        });
    });
    $(document).on("keydown", function(event) {
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

        if(rbt != null && direction != null){
            var rgb = rbt;

            rgb = rgb.replace(/[^\d,]/g, '').split(',');

            const hexColor = rgbToHex(rgb[0], rgb[1], rgb[2]);
            
            console.log("Pomakni" + hexColor + "robota u smjeru " + direction);
            if (move_robot(hexColor, direction)) {
                $.ajax({
                    url: './app/posaljiPotez.php',
                    type: 'GET',
                    data: {
                        potez: povuceniPotezi,
                        color: hexColor,
                        dir: direction,
                    },
                    success: function (data) {
                        if (typeof(data.error) !== "undefined") 
                            console.log("Greska:: posaljiPotez:: " + data.error);
                        // console.log("response of posaljiPoruku.php je: " + data.response);
                        povuceniPotezi++;
                        if (povuceniPotezi <= brojPoteza) {
                            if (dobroRijeseno()) {
                                disallowRobotMovement();
                                updatajRezultate(ime, 1);
                                licitacija();
                            }
                            else if (povuceniPotezi === brojPoteza) {
                                disallowRobotMovement();
                                vratiPoteze();
                                pomicanje();
                            }
                        } 
                    },
                    error: function( xhr, status ) {
                        console.log(xhr);
                        if( status !== null )
                            console.log( "FAIL (" + status + ")" );
                    }
                });

            }

            // Da li je proslo polje ODBINDANO??
            // OVO JE POTREBNO OPET POZVATI JER SE SADA MOZDA DOGODILA
            // PROMJENA POZICIJE I TO POLJE VIŠE NIJE BINDANO
            $( ".robot_field" ).each(function(index) {
                $(this).on("click", () => {
                    rbt = $(this).css('background-color');
                });
            });
        }


    });
    
}

function updatajRezultate(username, bodovi) {
    // console.log("updatam rezultate od: " + username + "za: " + bodovi);
    $.ajax({
        url: './app/updatajRezultate.php',
        type: 'GET',
        data: {
            username: username,
            bodovi: bodovi
        },
        success: function(data) {
            if (typeof(data.error) !== "undefined") 
                console.log("Greska:: posaljiPotez:: " + data.error);
            else    
                console.log("updatajRezultate response:: " + data.response);
        },
        error: function (xhr, status) {
            console.log(xhr)
            if( status !== null )
                console.log( "FAIL (" + status + ")" );
        }
    })
}

// Da li je trenutacni "token" dobro rijesen (ako je onda idemo u fazu licitacija, ako nije onda sljedeci igrac pokusava dati rijesenje).
// Treba slozit.
function dobroRijeseno() {
    return true;
    if (cilj.znak === null || cilj.boja === null)
        return false
    
    let class_name = "." + cilj.znak.replaceAll(" ", ".");

    let target_field = $(class_name).filter( function (){ 
        
        let color = $(this).css("color").replace(/[^\d,]/g, '').split(',');

        return rgbToHex(color[0], color[1], color[2]) === cilj.boja;
    }).get(0);

    let parent = $(target_field).parent().css("background-color").replace(/[^\d,]/g, '').split(',');

    return rgbToHex(parent[0], parent[1], parent[2]) === cilj.boja;
}

function disallowRobotMovement() {
    $( ".robot_field" ).each(function(index) {
        $(this).off();
    });
    $(document).off("keydown");
}

function waitOnHost(){
    console.log("cekam hosta da pokrene igru");
    $.ajax(
        {
            url: "./app/cekajTimer.php",
            type: "GET",
            // async: false,
            success: function( data ) 
            {
                // Sljedeća naredba ne treba: kako je dataType="json", data je već konvertiran iz stringa u objekt.
                // var data = JSON.parse( data );
                console.log("HOST je ready");

                if (typeof(data.error) !== "undefined") {
                    console.log("Greska:: cekajTimer.php:: " + data.error);
                }
                else {
                    $('body').show();
                    $( "#btn" ).on( "click", posaljiPoruku );
                    $("#txt").keypress(function(event) {
                        // console.log()
                        if (event.keyCode === 13 && !$("#btn").prop('disabled')) {
                            $("#btn").click();
                        }
                    });
                    cekajPoruku();
                    licitacija();
                }
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

// Sortira i sreduje licitaciju.
function srediLicitaciju(licitacija) {
    //console.log(licitacija);
    var glasovi = licitacija.split(',');
    var glasovi_sort = new Array(glasovi.length - 1);
    for (var i = 0; i < glasovi.length - 1; i++) {
        glasovi_sort[i] = glasovi[i].split(':'); 
    }
    
    glasovi_sort.sort(function(a, b) {
        var a1 = a[1]; var a0 = parseInt(a[0]);
        var b1 = b[1]; var b0 = parseInt(b[0]);
        if (a1 === b1) return b0 - a0;
        else if (a1 > b1) return 1;
        else return -1;
    })
    var sredeni_glasovi = [];
    for (var i = 0; i < glasovi_sort.length; i++) {
        if (i === 0 || glasovi_sort[i][1] !== sredeni_glasovi[sredeni_glasovi.length - 1][1]) {
            sredeni_glasovi.push(glasovi_sort[i]);
        }
    }
    
    sredeni_glasovi.sort( function(a, b) {
        var a0 = parseInt(a[0]);
        var a2 = parseInt(a[2]);
        var b0 = parseInt(b[0]);
        var b2 = parseInt(b[2]);

        if (a2 !== b2) return a2 - b2;
        return a0 - b0;
    })
    return sredeni_glasovi;
}

function dajLicitaciju() {
    var licitacija;
    $.ajax({
        url: './app/dajLog.php',
        data: {
            filename: 'licitacija.log'
        },
        type: "GET",
        // Asinkrono jer se radi o upitu za podatcima.
        async: false,
        success: function(data) {
            if( typeof( data.error ) !== "undefined" ) 
                console.log( "dajPoteze :: success :: server javio grešku " + data.error );
            licitacija = srediLicitaciju(data.licitacija);
        },
        error: function (xhr, status) {
            console.log(xhr);
        }
    })
    return licitacija;
}

function povuciPoteze(brojPoteza) {
    allowRobotMovement(brojPoteza);
    console.log("dozvoljeni potezi robota");
}

function cekajPotez(brojPoteza) {
    $.ajax({
        url: './app/cekajPotez.php',
        type: "GET",
        data: {
            timestamp: timestampPotez,

            cache: new Date().getTime()
        },
        success: function(data) {
            if( typeof( data.error ) !== "undefined" ) {
                // Ipak je došlo do greške!
                console.log( "cekajPotez :: success :: server javio grešku " + data.error );
            }
            else {
                timestampPotez = data.timestamp;
                console.log("primljeni su potezi za boju: " + data.hexColor + " i smjer: " + data.direction);
                move_robot(data.hexColor, data.direction);
                povuceniPotezi++;
                console.log("potez dobiven, povuceniPotezi:" + povuceniPotezi + ", brojPoteza: " + brojPoteza);
                if (povuceniPotezi <= brojPoteza) {
                    if (dobroRijeseno()) {
                        // TODO: update rezultate.
                        licitacija();
                    }
                    else if (povuceniPotezi === brojPoteza) {
                        vratiPoteze();
                        pomicanje();
                    }
                    else {
                        cekajPotez(brojPoteza);
                    } 
                } 
            }
        },
        error: function(xhr, status) {
            if (status === "timeout") 
                cekajPotez(brojPoteza);
            else if( status !== null )
                console.log( "FAIL cekajPoteze.php (" + status + ")" );
        }
    })
}

function updateResults() {
    console.log("updatam rezultate");
    $.ajax({
        url: './app/dajLog.php',
        type: "GET",
        data: {
            filename: 'rezultati.log'
        },
        async: false,
        succes: function(data) {
            console.log("uspjesno sam dobio rezultati.log")
            if (typeof(data.error) !== "undefined") 
                console.log("Greska:: ocistiPoruke.php:: " + data.error);
            else {
                console.log("Dobiveni rezultati su: " + data.rezultati);
                rezultati = data.rezultati.split(',');
                for (var i = 0; i < rezultati.length; i++ ) {
                    if (rezultati[i] === "") continue;
                    $("#result").append('<div>' + '<b>' + rezultati[i].split(':')[0] + '</b>: ' + rezultati[i].split(':')[1] + '</div>');
                }
            }
        },
        error: function (xhr, status) {
            console.log(xhr);
        }
    })

}

function licitacija() {
    console.log("licitacija");
    
    // brisanje chata i licitacija.
    $.ajax({
        url: "./app/ocistiLog.php",
        type: "GET",
        data: {
            filenames: "chat.log,licitacija.log"
        },
        // Moram pocistit prije nego ucitam sljedece podatke.
        async: false,
        success: function (data) {
            if (typeof(data.error) !== "undefined") 
                console.log("Greska:: ocistiPoruke.php:: " + data.error);
            else
                console.log("Poruke uspjesno pociscene");
        },
        error: function(xhr, status) {
            console.log(xhr);
        }
    })
    $("#chat").empty();
    $("#ranking").empty();
    $("#result").empty();
    updateResults();

    $("#btn").prop('disabled', false);

    // Odbrojavanje sekundi.
    postaviTimer(vrijemeZaLicitaciju);
    $("#timer").html(vrijemeZaLicitaciju.toString());
    timer = setInterval(function () {
        preostalo = parseInt($("#timer").html());
        if (preostalo > 0)
            {
            $("#timer").html((preostalo - 1).toString());
            if (preostalo <= 6)
                $("#timer").css("color","red")
            }
    }, 1000);


    $.ajax({
        url: "./app/cekajTimer.php",
        type: "GET",
        success: function (data) {
            if (typeof(data.error) !== "undefined") {
                console.log("Greska:: cekajTimer.php:: " + data.error);
            }
            else {
                odigrali = [];
                clearInterval(timer);
                $("#timer").css("color","black")
                $("#timer").html("0");
                $("#btn").prop('disabled', true);
                pomicanje();
            }
        }
    })
}

function pomicanje() {
    console.log("pomicanje");
    timestampPotez = Math.round(Date.now() / 1000);
    $.ajax({
        url: "./app/ocistiLog.php",
        type: "GET",
        data: {
            filenames: "potezi.log"
        },
        // Moram pocistit prije nego ucitam sljedece podatke.
        async: false,
        success: function (data) {
            if (typeof(data.error) !== "undefined") 
                console.log("Greska:: ocistiPoteze.php:: " + data.error);
            else
                console.log("Potezi uspjesno pociscene");
        },
        error: function(xhr, status) {
            console.log(xhr);
        }
    })
    var ranking = dajLicitaciju();
    var nekoJeIgral = false
    for (var i = 0; i < ranking.length; i++ ) {
        console.log(ranking[i][1]);
        if (odigrali.includes(ranking[i][1])) 
            continue;
        console.log(ranking[i][1] + " nije jos igrao.");
        odigrali.push(ranking[i][1]);
        nekoJeIgral = true;

        zapamtiPozicije();
        povuceniPotezi = 0;
        if (ime === ranking[i][1]) {
            console.log("povlacim " + ranking[i][2] + " poteza");
            povuciPoteze(parseInt(ranking[i][2]));
        }
        else {
            console.log("cekam poteze");
            cekajPotez(parseInt(ranking[i][2]));
        }
        break;
    }
    if (!nekoJeIgral) licitacija();
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
            timestamp: timestampPoruka, 

            // cache = svaki put šaljemo i trenutno vrijeme tako da browser ne pročita iz 
            //         cache-a odgovor servera, nego ga zaista ide kontaktirati.
            // Da smo koristili post, ovo ne bi bilo potrebno. (POST se ne cache-ira.)
            cache: new Date().getTime()
        },
        dataType: "json",
        success: function( data ) 
        {
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
                timestampPoruka = data.timestamp;
            
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
