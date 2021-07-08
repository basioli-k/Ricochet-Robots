<?php

// function shutdown()
// {
// 	require_once __SITE_PATH . "/view/gameplay.php";
// }

// register_shutdown_function('shutdown');

class GameplayController{

    function index() {

        if( !isset($_SESSION["game"] ) ){
            $board = [  [new Field("1100"), new Field("0110"), new Field("1100"), new Field("0100"), new Field("0101"), new Field("0100"), new Field("0100"), new Field("0100"), new Field("0100"), new Field("0110"), new Field("1100"), new Field("0100"), new Field("0100"), new Field("0100"), new Field("0101"), new Field("0110")],
				[new Field("1000"), new Field("0001", NULL, RED), new Field("0000"),new Field("0010"), new Field("1100",array(RED, MOON),NULL), new Field("0000"),new Field("0000"), new Field("0000"),new Field("0000"), new Field("0000"), new Field("0000"),new Field("0000"), new Field("0000"),new Field("0010"), new Field("1100",array(RED, GEAR),NULL), new Field("0010")],
				[new Field("1000"), new Field("0110",array(GREEN, GEAR),NULL), new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010"), new Field("1001",array(BLUE, STAR), NULL), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010")],
				[new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0011",array(YELLOW,STAR),NULL), new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0100"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0011")],
				[new Field("1000",NULL, YELLOW), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0100"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0110")],
				[new Field("1001"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010")],
				[new Field("1100"), new Field("0000"), new Field("0010"), new Field("1001",array(BLUE, PLANET), NULL), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0001"), new Field("0001"), new Field("0000"), new Field("0001"), new Field("0000"), new Field("0000"), new Field("0011",array(GREEN, MOON),NULL), new Field("1000"), new Field("0010")],
				[new Field("1000"), new Field("0000"), new Field("0000"), new Field("0100"), new Field("0000"), new Field("0001"), new Field("0010"), new Field("1100",NULL, BLACK), new Field("0110",NULL, BLACK), new Field("1000"), new Field("0110",array(YELLOW, PLANET),NULL), new Field("1000"), new Field("0000"), new Field("0100"), new Field("0000"), new Field("0010")],
				[new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0110",array(PURPLE, VORTEX), NULL), new Field("1010"), new Field("1001",NULL, BLACK), new Field("0011",NULL, BLACK), new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010")],
				[new Field("1000"), new Field("0011", array(BLUE, MOON), NULL), new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0100"), new Field("0100"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010"), new Field("1001",array(BLUE, GEAR),NULL), new Field("0000"), new Field("0010")],
				[new Field("1000"), new Field("0100"), new Field("0000"), new Field("0010"), new Field("1001",array(GREEN, PLANET),NULL), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010")],
				[new Field("1001"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0100"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0011",array(YELLOW, MOON),NULL), new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0011")],
				[new Field("1100"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0001"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0001"), new Field("0110")],
				[new Field("1000"), new Field("0000"), new Field("0000"), new Field("0001"), new Field("0000"), new Field("0110",array(RED, STAR),NULL), new Field("1000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0001"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0110",array(RED,PLANET),NULL), new Field("1010")],
				[new Field("1000",NULL, GREEN), new Field("0000"), new Field("0010"), new Field("1100",array(YELLOW, GEAR),NULL), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010"), new Field("1100",array(GREEN,STAR),NULL), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0000"), new Field("0010")],
				[new Field("1001"), new Field("0001"), new Field("0001"), new Field("0001"), new Field("0001"), new Field("0001"), new Field("0011"), new Field("1001"), new Field("0001"), new Field("0001"), new Field("0001"), new Field("0011"), new Field("1001"), new Field("0001",NULL, BLUE), new Field("0001"), new Field("0011")]
			];
            $game = new Game($board, []);

			$game->add_player($_SESSION["player"]);
            
            $_SESSION["game"] = $game;
        }

		$filename  = __SITE_PATH . "/controller/usernames.log";
		
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
			echo $error;
			exit;
		}
		$active_users = explode(',', file_get_contents($filename));
		$count = 0;
		foreach( $active_users as $user){
			if( isset($_SESSION["player"]) && $_SESSION["player"]->username !== trim($user))
				$count++;
		}
		echo $_SESSION["player"]->username . "<br/>";

		if ($count === count($active_users))
			file_put_contents( $filename,  $_SESSION["player"]->username . ",", FILE_APPEND);

        require_once __SITE_PATH . "/view/gameplay.php";
        
    }
}

?>