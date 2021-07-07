<?php require_once __SITE_PATH . "/view" . '/_header.php';
?>

<form method="post" action= "<?php echo __SITE_URL; ?>/index.php?rt=login/verifyLogin" >
    <label for="user"> Username: </label>
    <input type="text" name="username"> <br>
    <label for="password"> Password: </label>
    <input type="password" name="password"> <br>
    <button type="submit" name="rt" value="homepage"> Login </button>
</form>

<div>
    Click here to <a href="<?php echo __SITE_URL; ?>/index.php?rt=register">register</a>.
</div>

<?php
if(isset($warning) && strlen($warning) > 0)
    echo "<div>" . $warning . "</div>";
?>

<?php require_once __SITE_PATH . "/view" . '/_footer.php'; ?>
