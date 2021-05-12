<?php

require_once("field.php");
require_once("constants.php");
require_once("game.php");
require_once("player.php");

class Game{
	public $board;
	public $players;
	public $targets = array(array(RED, STAR), array(RED, PLANET), array(RED, GEAR), array(RED, MOON),
							array(BLUE, STAR), array(BLUE, PLANET), array(BLUE, GEAR), array(BLUE, MOON),
							array(GREEN, STAR), array(GREEN, PLANET), array(GREEN, GEAR), array(GREEN, MOON),
							array(YELLOW, STAR), array(YELLOW, PLANET), array(YELLOW, GEAR), array(YELLOW, MOON),
							array(BLACK, VORTEX));

	function __construct($board, $players){
		$this->board = $board;
		$this->players = $players;
	}

	function add_player($player){
		array_push($this->players, $player);
	}
};

if(!isset($game)){
	// order of walls is left, top, right, bottom
	// ex. 1101, walls are left, top, botom
	$board = [  [new Field("1101"), new Field("0100"), new Field("0110")],
				[new Field("1100"), new Field("0010", NULL, RED), new Field("1010")],
				[new Field("1001"), new Field("0001"), new Field("0011")]];

	$player = new Player("karlo");
	$game = new Game($board, array($player));
}

?>