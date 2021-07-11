<?php

function sendJSONandExit( $message )
{
    // Kao izlaz skripte pošalji $message u JSON formatu i prekini izvođenje.
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}


// File u kojem se nalazi samo zadnja poruka.
$filename  = 'potezi.log';

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

if( $error !== "" )
{
    $response = [];
    $response[ 'error' ] = $error;

    sendJSONandExit( $response );
}

$lastmodif    = isset( $_GET['timestamp'] ) ? $_GET['timestamp'] : 0;

// Otkrij kad je zadnji put bio promijenjena datoteka u kojoj je spremljena zadnja poruka.
$currentmodif = filemtime( $filename );
$potezi = file_get_contents($filename);

// Petlja koja se vrti sve dok se datoteka ne promijeni
while( $currentmodif <= $lastmodif || $potezi === "")
{
    usleep( 10000 ); // odspavaj 10ms da CPU malo odmori :)
    clearstatcache();
    $currentmodif = filemtime( $filename ); // ponovno dohvati vrijeme zadnje promjene datoteke
    $potezi = file_get_contents($filename);
}

$poteziArray = explode(',', $potezi);
$zadnjiPotez = $poteziArray[count($poteziArray)-2];
$hexColor = explode(':', $zadnjiPotez)[0];
$direction = explode(':', $zadnjiPotez)[1];


$response = array();
$response[ 'hexColor' ] = $hexColor;
$response[ 'direction' ] = $direction;
$response[ 'potezi' ] = $potezi;
$response[ 'timestamp' ] = $currentmodif;

sendJSONandExit( $response );

?>