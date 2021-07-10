<?php
/*
    Ova skripta nam služi i za spremanje novih poruka koje klijent šalje
    i za slanje poruka klijentu koje je netko drugi napisao.
*/

function sendJSONandExit( $message )
{
    // Kao izlaz skripte pošalji $message u JSON formatu i prekini izvođenje.
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}


// File u kojem se nalazi samo zadnja poruka.
$filename  = 'chat.log';

$error = "";
if( !file_exists( $filename ) )
    $error = $error . "Datoteka " . $filename . " ne postoji. ";
else
{
    if( !is_readable( $filename ) )
        $error = $error . "Ne mogu čitati iz datoteke " . $filename . ". ";

    if( !is_writable( $filename ) )
        $error = $error . "Ne mogu pisati u datoteku " . $filename . ". ";
}

$licitacija = 'licitacija.log';

if( !file_exists( $licitacija ) )
    $error = $error . "Datoteka " . $licitacija . " ne postoji. ";
else
{
    if( !is_readable( $licitacija ) )
        $error = $error . "Ne mogu čitati iz datoteke " . $licitacija . ". ";

    if( !is_writable( $licitacija ) )
        $error = $error . "Ne mogu pisati u datoteku " . $licitacija . ". ";
} 

if( $error !== "" )
{
    $response = [];
    $response[ 'error' ] = $error;

    sendJSONandExit( $response );
}



// Sad ide dio koda koji detektira je li netko drugi napisao poruku.
// Ovo je "long polling" u skripti.

// Klijent će u zahtjevu poslati svoje trenutno vrijeme.
$lastmodif    = isset( $_GET['timestamp'] ) ? $_GET['timestamp'] : 0;

// Otkrij kad je zadnji put bio promijenjena datoteka u kojoj je spremljena zadnja poruka.
$currentmodif = filemtime( $filename );

// Petlja koja se vrti sve dok se datoteka ne promijeni
while( $currentmodif <= $lastmodif )
{
    usleep( 10000 ); // odspavaj 10ms da CPU malo odmori :)
    clearstatcache();
    $currentmodif = filemtime( $filename ); // ponovno dohvati vrijeme zadnje promjene datoteke
}

// Kad dođemo do ovdje, znamo da je datoteka bila promijenjena.
// Spremi njen sadržaj u $response[ 'msg' ] i vrijeme zadnje promjene u $response[ 'timestamp' ]
$response = array();
$response[ 'licitacija' ] = file_get_contents( $licitacija);
$response[ 'msg' ]       = file_get_contents( $filename );
$response[ 'timestamp' ] = $currentmodif;

// Napravi JSON string od ovog i ispiši ga (tj. pošalji klijentu).
sendJSONandExit( $response );

?>