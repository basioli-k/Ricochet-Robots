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


if( $error !== "" )
{
    $response = [];
    $response[ 'error' ] = $error;

    sendJSONandExit( $response );
}


// Ako klijent šalje novu poruku, pospremi ju u datoteku
$msg = isset( $_GET['msg'] ) ? $_GET['msg'] : '';
$ime = isset( $_GET['ime'] ) ? $_GET['ime'] : '';

if( $msg != '' && $ime != '' ) // nama će ime uvijek biti valjano
{
    // Spremi poruku u datoteku (ovo će prebrisati njen sadržaj)
    file_put_contents( $filename, '<b>' . $ime . '</b>: ' . $msg );

    // Iako klijent zapravo ne treba odgovor kada šalje novu poruku,
    // možemo mu svejeno nešto odgovoriti da olakšamo debugiranje na strani klijenta.
    $response = [];
    $response[ 'ime' ] = $ime;
    $response[ 'msg' ] = $msg;
    sendJSONandExit( $response );
}
else
{
    $response = [];
    $response[ 'error' ] = "Poruka nema definirano polje ime ili polje msg.";

    sendJSONandExit( $response );
}

?>