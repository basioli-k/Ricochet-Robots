<?php 
require_once '../misc/constants.php';

function shuffle_tokens(){
    $tokens = array(array(RED, STAR), array(RED, PLANET), array(RED, GEAR), array(RED, MOON),
                    array(BLUE, STAR), array(BLUE, PLANET), array(BLUE, GEAR), array(BLUE, MOON),
                    array(GREEN, STAR), array(GREEN, PLANET), array(GREEN, GEAR), array(GREEN, MOON),
                    array(YELLOW, STAR), array(YELLOW, PLANET), array(YELLOW, GEAR), array(YELLOW, MOON),
                    array(PURPLE, VORTEX));
    
    
    shuffle($tokens);
    
    $filename = "./tokens.log";

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

    // echo "prije errora " . $error;
    if( $error !== "" )
    {
        $response = [];
        $response[ 'error' ] = $error;

        sendJSONandExit( $response );
    }
    $content = "";

    for ($i = 0 ; $i < count($tokens); ++$i){
        // print_r($tokens[$i][0]);
        $content .= $tokens[$i][0] . ":" . $tokens[$i][1] . ",";
    }

    file_put_contents($filename, $content);
}

function sendJSONandExit( $message )
{
    // Kao izlaz skripte pošalji $message u JSON formatu i prekini izvođenje.
    header( 'Content-type:application/json;charset=utf-8' );
    echo json_encode( $message );
    flush();
    exit( 0 );
}

$filename  = "../controller/usernames.log";

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

// echo "prije errora " . $error;
if( $error !== "" )
{
    $response = [];
    $response[ 'error' ] = $error;

    sendJSONandExit( $response );
}

$msg = file_get_contents($filename);
// echo "tu sam" . $msg;
if (count(explode(",", $msg)) === 2) {
    shuffle_tokens();
    $response = [];
    $response[ 'prvi' ] = true;
    sendJSONandExit( $response );
} else {
    $response = [];
    $response[ 'prvi' ] = false;
    sendJSONandExit( $response );
}

?>