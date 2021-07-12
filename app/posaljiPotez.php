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

// Ako klijent šalje novu poruku, pospremi ju u datoteku
$potez = isset( $_GET['potez'] ) ? $_GET['potez'] : '';
$color = isset( $_GET['color'] ) ? $_GET['color'] : '';
$dir = isset( $_GET['dir'] ) ? $_GET['dir'] : '';



if( $color != '' && $dir != '' && $potez != '') // nama će ime uvijek biti valjano
{
    $obrisano = 'nijeObrisano';
    if ($potez === '0') {
        file_put_contents( $filename, "");
        $obrisano = 'jestObrisano';
    }

    // Spremi poruku u datoteku (ovo će prebrisati njen sadržaj)
    file_put_contents( $filename, $color . ':' . $dir . ',', FILE_APPEND );

    // Iako klijent zapravo ne treba odgovor kada šalje novu poruku,
    // možemo mu svejeno nešto odgovoriti da olakšamo debugiranje na strani klijenta.
    $response = [];
    $response[ 'response' ] = "potezi su poslani" . $obrisano;
    sendJSONandExit( $response );
}
else
{
    $response = [];
    $response[ 'error' ] = "Poruka nema definiranu broj poteza ili boju ili smjer kretenje";

    sendJSONandExit( $response );
}

?>