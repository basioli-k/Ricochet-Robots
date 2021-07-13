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
$filename  = 'rezultati.log';

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

// Ako klijent šalje novu poruku, pospremi ju u datoteku
$username = isset( $_GET['username'] ) ? $_GET['username'] : '';
$bodovi = isset( $_GET['bodovi'] ) ? $_GET['bodovi'] : '';



if( $username != '' && $bodovi != '') 
{
    $rezultati_string = file_get_contents($filename);
    $rezultati = explode(',', $rezultati_string);
    array_pop($rezultati);
    $postavljeni = false;
    for ($i = 0; $i < count($rezultati); $i++) {
        $ime = explode(':', $rezultati[$i])[0];
        if ($ime === $username) {
            $broj = explode(':', $rezultati[$i])[1];
            $broj += $bodovi;
            $rezultati[$i] = $ime . ':' . $broj;
            $postavljeni = true;
        }
    }
    if (!$postavljeni) {
        array_push($rezultati, $username . ':' . $bodovi);
    }

    $rezultati_string = "";

    for ($i = 0; $i < count($rezultati); $i++) {
        $rezultati_string = $rezultati_string . $rezultati[$i] . ',';
    }

    file_put_contents($filename, $rezultati_string);

    $response = [];
    $response[ 'response' ] = "Rezultati su updatani na: " . $rezultati_string;
    sendJSONandExit( $response );
}
else
{
    $response = [];
    $response[ 'error' ] = "Poruka nema definirana polja username ili bodovi";

    sendJSONandExit( $response );
}

?>