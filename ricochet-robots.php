<?php
require_once("game.php");
require_once("field.php");
require_once("constants.php");

session_start();

if( !isset($_SESSION["game"] ) ){
	
	$_SESSION["game"] = $game; //ovo je u game.php
}

if(isset($_POST["robot"]) && isset($_POST["direction"]) && isset($_SESSION["game"])){

	$board = $_SESSION["game"]->board;
	$robot = $_POST["robot"];
	$direction = $_POST["direction"];
	$_SESSION["game"]->players[0]->move_robot( $board, $robot, $direction);
	$_SESSION["game"]->board = $board;
}

display();

function display(){
	?>
	<!DOCTYPE html>

	<html>
		<head>
			<meta charset="utf-8">
			<title>Ricochet robots</title>

			<link rel="stylesheet" href="">
		</head>
		<body>
			<?php draw_board() ?>
			<br>
			<form action="ricochet-robots.php" method="post">
				<!-- bolje da biramo klikom na robota na ploci bas -->
				<select name="robot">
					<option value=<?php echo RED; ?> selected> Red </option>
					<option value=<?php echo BLUE; ?>> Blue </option>
					<option value=<?php echo GREEN; ?>> Green </option>
					<option value=<?php echo YELLOW; ?>> Yellow </option>
				</select>
				<br>
				<input name = "direction" type="submit" value="left">
				<input name = "direction" type="submit" value="top">
				<input name = "direction" type="submit" value="right">
				<input name = "direction" type="submit" value="bottom">

			</form>
		</body>
	</html>
	<?php

}

function draw_board(){
	$board = $_SESSION["game"]->board;
	?>
	<table style= "font-size: x-large; width: 200px; border-collapse: collapse;">
	<?php
	for($i = 0 ; $i < count($board); $i++){
		?>
		<tr>
		<?php
		for($j = 0 ; $j < count($board[$i]); $j++){
			$board[$i][$j]->draw_field();
		}
		?>
		</tr>
		<?php
	}
	?>
	</table>
	<?php
}
?>