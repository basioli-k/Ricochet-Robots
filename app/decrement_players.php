<?php
function sendJSONandExit( $message )
{
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}

$filename  = "./player_count.log";

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

$active_players = (int) file_get_contents($filename) - 1;

file_put_contents( $filename,  $active_players);

$response["player_num"] = $active_players;
sendJSONandExit( $response );

?>